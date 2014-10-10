(ns chapter-tracker.view.series-and-episode-panel (:gen-class)
  (:use [chapter-tracker.view tools episode-table create-series-dialog])
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(java.awt.event MouseAdapter)
  '(javax.swing JPanel JList JTable JScrollPane)
  '(javax.swing.event ListSelectionListener)
)

(defn create-series-and-episode-panel[update-episode-panel-function update-series-directories-panel-function]
  (let [serieses-list (JList.)
        serieses-scroll-pane (JScrollPane. serieses-list)
        episodes-table (create-episodes-table update-episode-panel-function)
        episodes-scroll-pane (JScrollPane. episodes-table)
        update-serieses-list-function #(.setListData serieses-list (to-array (fetch-series-records)))
        get-selected-series-function #(.getSelectedValue serieses-list)
        update-episode-table-function #(do
                                         (let [series-record (.getSelectedValue serieses-list)]
                                          (update-episodes-table episodes-table series-record)
                                          (update-series-directories-panel-function series-record))
                                        (update-episode-panel-function nil nil))
       ]
    [
     (create-panel {:width 600 :height 400}
                   (update-serieses-list-function)
                   (.setFixedCellWidth serieses-list 180)
                   (add-with-constraints serieses-scroll-pane
                                         (gridx 0) (gridy 0) (fill GridBagConstraints/BOTH)
                   )
                   (.setPreferredSize episodes-scroll-pane (Dimension. 400 400))
                   ;(.setSize episodes-scroll-pane (Dimension. 200 200))
                   ;(println (.getWidth episodes-scroll-pane))
                   (add-with-constraints episodes-scroll-pane
                                         (gridx 1) (gridy 0) (fill GridBagConstraints/BOTH)
                   )
                   (.addListSelectionListener serieses-list (proxy [ListSelectionListener] []
                                                              (valueChanged [e]
                                                                (when-not (.getValueIsAdjusting e)
                                                                  (update-episode-table-function)
                                                                )
                                                              )))

                   (.addMouseListener serieses-list (proxy [MouseAdapter] []
                                                      (mouseClicked [e]
                                                        (when (= 2 (.getClickCount e))
                                                          (.show (create-create-series-dialog
                                                                   update-serieses-list-function
                                                                   (-> serieses-list .getSelectedValue :series-id)
                                                                 ))
                                                        )
                                                          )))
     )
     update-serieses-list-function
     update-episode-table-function
     get-selected-series-function
    ]
  )
)

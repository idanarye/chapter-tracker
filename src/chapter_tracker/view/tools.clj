(ns chapter-tracker.view.tools)

(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(java.awt.event ActionListener)
  '(javax.swing JFrame JPanel JButton JFileChooser)
  '(javax.swing.table DefaultTableModel TableCellRenderer TableCellEditor)
)

(defmacro create-frame [properties & body]
  `(let [~'container (JFrame.)
         ~'frame ~'container
         layout# (GridBagLayout.)]
     ~(if (contains? properties :title) `(.setTitle ~'container ~(:title properties)))
     (.setLayout ~'container layout#)
     ~(if (or (contains? properties :width) (contains? properties :height))
        `(.setPreferredSize ~'container (Dimension.
                                          ~(if (contains? properties :width) (:width properties) `(.getWidth ~'container))
                                          ~(if (contains? properties :height) (:height properties) `(.getHeight ~'container))
                                        ))
      )
     ~@body
     (.layoutContainer layout# ~'container)
     (.pack ~'frame)
     ~'container
   )
)

(defmacro create-panel [properties & body]
  `(let [~'container (JPanel.)
         ~'panel ~'container
         layout# (GridBagLayout.)]
     (.setLayout ~'container layout#)
     (.setBorder ~'panel (javax.swing.BorderFactory/createLineBorder java.awt.Color/LIGHT_GRAY))
     ~(if (or (contains? properties :width) (contains? properties :height))
        `(.setPreferredSize ~'container (Dimension.
                                          ~(if (contains? properties :width) (:width properties) `(.getWidth ~'container))
                                          ~(if (contains? properties :height) (:height properties) `(.getHeight ~'container))
                                        ))
      )
     ~@body
     (.layoutContainer layout# ~'container)
     ~'container
   )
)

(defmacro add-with-constraints [component & constraints]
  (let [grid-bag-constraints (gensym)]
    `(.add ~'container ~component (let [~grid-bag-constraints (GridBagConstraints.)]
                                    (set! (. ~grid-bag-constraints anchor) GridBagConstraints/NORTHWEST)
                                    ~@(map (fn [[method arg]]
                                             `(set! (. ~grid-bag-constraints ~method) ~arg)
                                           ) constraints)
                                    ~grid-bag-constraints
                                  )
     )
  )
)

(defmacro action-button [caption & body]
  `(let [result# (JButton. ~caption)]
     (.addActionListener result# (proxy [ActionListener] []
                                   (actionPerformed [e#]
                                     ~@body
                                   )
                                 ))
     result#
   )
)

(defn choose-*
  ([target-type] (choose-* target-type "."))
  ([target-type start-from]
   (let [chooser (JFileChooser. start-from)]
     (.setFileSelectionMode chooser target-type)
     (condp = (.showOpenDialog chooser nil)
       JFileChooser/APPROVE_OPTION (.. chooser getSelectedFile toString)
       JFileChooser/CANCEL_OPTION nil
     )
   ))
)

(defn choose-dir
  ([] (choose-* JFileChooser/DIRECTORIES_ONLY))
  ([start-from] (choose-* JFileChooser/DIRECTORIES_ONLY start-from))
)
(defn choose-file
  ([] (choose-* JFileChooser/FILES_ONLY))
  ([start-from] (choose-* JFileChooser/FILES_ONLY start-from))
)
